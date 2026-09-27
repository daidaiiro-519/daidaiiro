<?php
use PHPUnit\Framework\TestCase;
use PHPUnit\Framework\Attributes\DataProvider;
require_once __DIR__ . '/../src/Order.php';
function scenario(string $id): void { $p = getenv("SCENARIO_TRACE"); if ($p) file_put_contents($p, $id . "\n", FILE_APPEND); }
class OrderTest extends TestCase {
    public static function cases(): array { return [[[], "SC-01J7Q4M", false], [["りんご"], "SC-01J7Q4N", true]]; }
    #[DataProvider('cases')]
    public function testConfirm(array $lines, string $id, bool $ok): void {
        scenario($id);
        $o = new Order(); $o->lines = $lines;
        if ($ok) { $o->confirm(); $this->assertSame("確定済", $o->status); }
        else { $this->expectException(RuntimeException::class); $o->confirm(); }
    }
}
