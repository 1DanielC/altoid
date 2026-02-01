import {logError, logInfo, logWarning} from "../services/log.ts";
export default function TestButton() {
  return (
      <button
          className="button"
          onClick={() => {
            logInfo("POOP")
            logWarning("SCOOP")
            logError("DOOP")
          }}
      >
        Test Button
      </button>
  );
}
