import { receive } from "../usecase/receive";
export const handler = () => receive({ no: 1, verdict: "approve" });
