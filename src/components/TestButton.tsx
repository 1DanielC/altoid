import {useCameraQuery} from "../contexts/AppContext";
export default function TestButton() {
  const {refetch} = useCameraQuery();

  return (
      <button
          className="button"
          onClick={() => {
            console.log(refetch())
          }}
      >
        Test Button
      </button>
  );
}
