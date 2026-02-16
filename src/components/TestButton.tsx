import {getUser} from "../contexts/services/ApiService.ts";
export default function TestButton() {
  return (
      <button
          className="button"
          onClick={() => {
            getUser().then(r => console.log(r))
          }}
      >
        Test Button
      </button>
  );
}
