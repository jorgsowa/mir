===description===
does not report called private method
===file===
<?php
class Foo {
    public function run(): void {
        $this->helper();
    }

    private function helper(): void {}
}
===expect===
===ignore===
TODO
