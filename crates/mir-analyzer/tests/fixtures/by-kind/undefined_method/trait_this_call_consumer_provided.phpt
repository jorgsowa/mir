===description===
$this-> and static:: calls inside a trait body may be provided by the
consuming class — not UndefinedMethod when unresolved in the trait itself
===config===
suppress=UnusedParam
===file===
<?php
trait AssertsThings {
    public function assertOk(): void {
        $this->assertStatus(200);
    }

    public static function twice() {
        return static::range(1, 2);
    }
}

class UsesIt {
    use AssertsThings;

    public function assertStatus(int $code): void {}

    public static function range(int $a, int $b): array { return [$a, $b]; }
}
===expect===
