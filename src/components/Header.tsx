import LoginButton from './LoginButton';

export default function Header({ onOpenSettings }: { onOpenSettings: () => void }) {
  return (
    <div id="header">
      <span>OpenSpace Desktop Sync</span>
      <LoginButton onClick={onOpenSettings} />
    </div>
  );
}
