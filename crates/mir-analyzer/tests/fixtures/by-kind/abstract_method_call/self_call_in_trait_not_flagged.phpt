===description===
`self::` inside a trait names the using class, which must implement the trait's abstract static methods.
===file===
<?php
trait Schedules {
    abstract protected static function schedule(int $id): void;

    public function run(): void {
        self::schedule(1);
    }
}
final class Runner {
    use Schedules;

    protected static function schedule(int $id): void {}
}
