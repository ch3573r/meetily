'use client';

import React from 'react';
import { useSidebar } from '@/components/Sidebar/SidebarProvider';

interface MainContentProps {
  children: React.ReactNode;
}

const MainContent: React.FC<MainContentProps> = ({ children }) => {
  const { isCollapsed } = useSidebar();

  return (
    <main
      className={`h-[calc(100vh-var(--titlebar-height))] min-h-0 flex-1 overflow-hidden transition-all duration-300 ${
        isCollapsed ? 'ml-16' : 'ml-[17.5rem]'
      }`}
    >
      {children}
    </main>
  );
};

export default MainContent;
