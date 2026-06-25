===description===
NonStaticSelfCall does NOT fire when a non-static method is called via self:: from another non-static method.
===file===
<?php
class Timer {
    public function tick(): void {}

    public function tickTwice(): void {
        self::tick();
    }
}
===expect===
