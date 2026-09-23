export const x = 1;
/* import { y } from "../adapters/http";  ← 塊のコメントの中 */
const msg = "see ../adapters/http for details";
const lazy = async () => await import("../adapters/http");
