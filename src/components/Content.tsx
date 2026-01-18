import UploadList from './UploadList';
import UploadButton from './UploadButton';
import ClearCacheButton from './ClearCacheButton';
import TestButton from "./TestButton.tsx";

export default function Content() {
  return (
    <div id="content">
      <div className="content-container">
        <UploadList />
        <UploadButton />
        <ClearCacheButton />
        <TestButton />
      </div>
    </div>
  );
}
