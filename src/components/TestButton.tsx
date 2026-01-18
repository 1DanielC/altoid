import { request } from "../contexts/services/ApiService.ts";
import { useUserQuery } from "../contexts/AppContext";

export default function TestButton() {
  const { data: userInfo } = useUserQuery();

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
