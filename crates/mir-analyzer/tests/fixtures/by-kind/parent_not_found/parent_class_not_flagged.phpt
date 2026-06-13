===description===
ParentNotFound does NOT fire when the class has a parent.
===file===
<?php
class Base {
    public function build(): void {}
}

class Child extends Base {
    public function build(): void {
        parent::build();
    }
}
===expect===
