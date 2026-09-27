export class Order {
  constructor(lines = []) { this.status = "準備中"; this.lines = lines; }
  confirm() {
    if (this.lines.length === 0) throw new Error("明細が0件");
    this.status = "確定済";
  }
}
