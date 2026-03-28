import { getCurrentWindow } from '@tauri-apps/api/window';
import LoginButton from './LoginButton';

export default function Header({ onOpenSettings }: { onOpenSettings: () => void }) {
  const handleMouseDown = (e: React.MouseEvent) => {
    // Only drag from the header itself, not from buttons
    if ((e.target as HTMLElement).closest('button')) return;
    getCurrentWindow().startDragging();
  };

  return (
    <div id="header" onMouseDown={handleMouseDown}>
      <span>OpenSpace Desktop Sync</span>
      <LoginButton onClick={onOpenSettings} />
    </div>
  );
}
