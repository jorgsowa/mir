===description===
This in static closure
===file===
<?php
class C {
    public string $a = "zzz";
    public function f(): void {
        $f = static function (): void {
            echo $this->a;
        };
        $f();
    }
}

===expect===
InvalidScope@6:18-6:23: $this cannot be used in a static method
