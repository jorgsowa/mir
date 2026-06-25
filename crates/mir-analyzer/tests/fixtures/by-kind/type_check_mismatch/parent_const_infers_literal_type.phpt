===description===
parent::CONST resolves to the parent class constant's literal type.
===config===
suppress=UnusedVariable
===file===
<?php
class Base {
    const MAX = 100;
    const PREFIX = 'app_';
}
class Child extends Base {
    public function check(): void {
        $m = parent::MAX;
        /** @mir-check $m is 100 */
        $_ = $m;

        $p = parent::PREFIX;
        /** @mir-check $p is 'app_' */
        $_ = $p;
    }
}
===expect===
