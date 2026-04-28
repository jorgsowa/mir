===file===
<?php
class Base {
    /** @return string */
    public function describe(): string { return ''; }
}

class Child extends Base {
    /** @inheritDoc */
    public function describe(): string { return 'child'; }
}
===expect===
