import {request} from "../contexts/services/ApiService.ts";
import {useUser} from "../contexts/AppContext.tsx";

export default function TestButton() {
  const { userInfo } = useUser();
  return (
      <button
          className="button"
          onClick={() => runTest(userInfo)}
      >
        Test Button
      </button>
  );
}

const runTest = (userInfo: unknown) => {
  console.log("test", userInfo);
  request("GET", "/api/self", {}).then(r => {
    console.log("User info", r);
  });
}
