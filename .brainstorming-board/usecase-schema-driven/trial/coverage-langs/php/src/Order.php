<?php
class Order {
    public string $status = "準備中";
    public array $lines = [];
    public function confirm(): void {
        if (count($this->lines) === 0) throw new RuntimeException("明細が0件");
        $this->status = "確定済";
    }
}
