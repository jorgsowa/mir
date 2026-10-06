===description===
This in static closure
===file===
<?php
class C {
    public string $a = "zzz";
    public function f(): void {
        $f = static function (): void {
            echo $this->a;
//               ^^^^^ InvalidScope: $this cannot be used in a static method
        };
        $f();
    }
}
