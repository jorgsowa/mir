===description===
widened visibility not reported
===file===
<?php
class ParentClass {
    protected function doStuff(): void {}
}
class Child extends ParentClass {
    public function doStuff(): void {}
}
===expect===
===ignore===
TODO
