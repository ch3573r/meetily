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
      className={`min-h-screen flex-1 transition-all duration-300 ${
        isCollapsed ? 'ml-16' : 'ml-[17.5rem]'
      }`}
    >
      {children}
    </main>
  );
};

export default MainContent;
