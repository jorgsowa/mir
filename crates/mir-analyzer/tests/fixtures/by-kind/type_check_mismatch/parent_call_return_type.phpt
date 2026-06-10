===description===
Return type of parent:: call is correctly inferred
===file===
<?php
class Base {
    public function getFoo(): string { return "x"; }
}
class Child extends Base {
    public function test(): void {
        $result = parent::getFoo();
        /** @mir-check $result is string */
        $_ = $result;
    }
}
===expect===
