import {request} from "../rust-api/services/ApiService.ts";

export default function TestButton() {
  return (
      <button
          className="button"
          onClick={runTest}
      >
      </button>
  );
}

const runTest = () => {
  console.log("test");
  request("GET", "/api/self", {a: "b"}).then(r => {
    console.log("got data");
    console.log(r);
  });
}
