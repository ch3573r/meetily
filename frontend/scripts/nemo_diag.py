#!/usr/bin/env python3
"""
Nemotron DirectML vs CPU per-node diff for the encoder.

Pins ONE fp16 encoder + ONE input, exposes every internal tensor as a graph
output, runs the SAME input on two execution providers, and reports the FIRST
node (topological order) whose output diverges beyond tolerance. Also dumps
provider placement so we know DML is actually executing the encoder (vs silent
CPU fallback at partition boundaries).

Windows usage (on the GPU box):
    pip install onnxruntime-directml onnx numpy soundfile librosa
    python nemo_diag.py <model_dir> [wav]   # model_dir holds encoder.onnx(+.data)

It compares CPUExecutionProvider vs DmlExecutionProvider by default. On a box
without DML it falls back to CPU-vs-CPU (machinery sanity check; all diffs ~0).
"""
import argparse, os, sys, numpy as np

MELS, NFFT, HOP, WIN, FMAX, PRE, SR = 128, 512, 160, 400, 8000.0, 0.97, 16000
LOGF, CHUNK, NUMPROMPTS = 1.0 / (1 << 24), 32, 128

def make_input(model_dir, wav):
    import onnx
    # mel for window 0 (librosa, matches the verified pipeline) or synthetic.
    if wav and os.path.exists(wav):
        import librosa
        y, _ = librosa.load(wav, sr=SR, mono=True)
        emph = np.copy(y); emph[1:] = y[1:] - PRE * y[:-1]
        S = librosa.feature.melspectrogram(y=emph, sr=SR, n_fft=NFFT, hop_length=HOP,
            win_length=WIN, window='hann', center=True, n_mels=MELS, fmin=0, fmax=FMAX,
            power=2.0, htk=False, norm='slaney')
        mel = np.log(S + LOGF).astype(np.float32)[:, :CHUNK]
        if mel.shape[1] < CHUNK:
            mel = np.pad(mel, ((0, 0), (0, CHUNK - mel.shape[1])))
        a = mel[None, :, :]
    else:
        a = (np.sin(np.arange(MELS)[:, None] * 0.13 + np.arange(CHUNK)[None, :] * 0.31) * 3.0)[None].astype(np.float32)
    # match encoder input dtypes
    m = onnx.load(os.path.join(model_dir, "encoder.onnx"), load_external_data=False)
    et = {i.name: i.type.tensor_type.elem_type for i in m.graph.input}
    ii = lambda n, v: (np.array(v, np.int32) if et.get(n) == onnx.TensorProto.INT32 else np.array(v, np.int64))
    lang = np.zeros((1, NUMPROMPTS), np.float32); lang[0, 0] = 1.0
    feeds = {
        "audio_signal": a.astype(np.float32),
        "audio_length": ii("audio_length", [CHUNK]),
        "language_mask": lang,
        "pre_cache": np.zeros((1, MELS, 9), np.float32),
        "cache_last_channel": np.zeros((24, 1, 56, 1024), np.float32),
        "cache_last_time": np.zeros((24, 1, 1024, 8), np.float32),
        "cache_last_channel_len": ii("cache_last_channel_len", [0]),
    }
    return {k: v for k, v in feeds.items() if k in et}

def expose_all_outputs(src, dst):
    import onnx
    # Do not load the 1.2 GB external weights into this diagnostic graph. Keeping
    # the existing external-data references makes _encoder_allout.onnx small and
    # keeps it pointing at encoder.onnx.data in the same directory.
    m = onnx.load(src, load_external_data=False)
    existing = {o.name for o in m.graph.output}
    names = []
    producers = {}
    for node in m.graph.node:
        if node.op_type == "Constant":
            continue
        for o in node.output:
            if o and o not in existing:
                names.append(o)
            if o:
                producers[o] = {
                    "node_name": node.name or "<unnamed>",
                    "op_type": node.op_type,
                }
    for n in names:
        m.graph.output.extend([onnx.helper.make_empty_tensor_value_info(n)])
    onnx.save(m, dst)
    return names, producers

def make_session_options(opt):
    import onnxruntime as ort
    so = ort.SessionOptions()
    so.log_severity_level = 0  # VERBOSE: provider placement is logged here.
    so.enable_mem_pattern = False
    so.execution_mode = ort.ExecutionMode.ORT_SEQUENTIAL
    levels = {
        "disable": ort.GraphOptimizationLevel.ORT_DISABLE_ALL,
        "basic": ort.GraphOptimizationLevel.ORT_ENABLE_BASIC,
        "extended": ort.GraphOptimizationLevel.ORT_ENABLE_EXTENDED,
        "all": ort.GraphOptimizationLevel.ORT_ENABLE_ALL,
    }
    so.graph_optimization_level = levels[opt]
    return so

def run(model_dir, wav, opt):
    import onnxruntime as ort
    feeds = make_input(model_dir, wav)
    avail = ort.get_available_providers()
    print("available providers:", avail)
    pa = 'CPUExecutionProvider'
    has_dml = 'DmlExecutionProvider' in avail
    providers_b = ['DmlExecutionProvider', 'CPUExecutionProvider'] if has_dml else ['CPUExecutionProvider']
    print(f"comparing A={[pa]}  vs  B={providers_b}  opt={opt}")

    exposed = os.path.join(model_dir, "_encoder_allout.onnx")
    names, producers = expose_all_outputs(os.path.join(model_dir, "encoder.onnx"), exposed)
    print(f"exposed {len(names)} intermediate tensors")

    # provider placement: run with VERBOSE logging once and grep stderr for
    # "Node placements" / "placed on" to confirm DML actually executes the
    # encoder (vs silent CPU fallback at partition boundaries).
    sa = ort.InferenceSession(exposed, sess_options=make_session_options(opt), providers=[pa])
    sb = ort.InferenceSession(exposed, sess_options=make_session_options(opt), providers=providers_b)
    print("B session providers:", sb.get_providers())
    out_names = [o.name for o in sa.get_outputs()]
    ra = dict(zip(out_names, sa.run(out_names, feeds)))
    rb = dict(zip(out_names, sb.run(out_names, feeds)))

    print(f"\n{'node output':50} {'op':18} {'max_abs_err':>12} {'cosine':>8} {'A|max|':>8} {'B|max|':>8}")
    first = None
    for n in out_names:
        x, y = ra[n], rb[n]
        if x.shape != y.shape or x.dtype.kind != 'f':
            continue
        xf, yf = x.astype(np.float64).ravel(), y.astype(np.float64).ravel()
        mae = float(np.abs(xf - yf).max()) if xf.size else 0.0
        denom = (np.linalg.norm(xf) * np.linalg.norm(yf)) or 1.0
        cos = float(np.dot(xf, yf) / denom)
        amax = float(np.abs(xf).max()) if xf.size else 0.0
        bmax = float(np.abs(yf).max()) if yf.size else 0.0
        flag = (mae > 1e-2 and cos < 0.99)
        if flag and first is None:
            first = n
        op = producers.get(n, {}).get("op_type", "")
        if flag or n == "encoded_output" or n == out_names[-1]:
            print(f"{n[:50]:50} {op[:18]:18} {mae:12.4f} {cos:8.4f} {amax:8.3f} {bmax:8.3f}{'  <-- FIRST DIVERGENCE' if n==first else ''}")
    if first:
        p = producers.get(first, {})
        print(f"\nFIRST DIVERGENT NODE: output={first} op={p.get('op_type')} node={p.get('node_name')}")
    else:
        print("\nFIRST DIVERGENT NODE: None")

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("model_dir")
    parser.add_argument("wav", nargs="?")
    parser.add_argument("--opt", choices=["disable", "basic", "extended", "all"], default="disable",
                        help="Graph optimization level for both sessions. Default mirrors the app's Nemotron DirectML probe.")
    args = parser.parse_args()
    run(args.model_dir, args.wav, args.opt)
