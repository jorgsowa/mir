===description===
does not report this method call
===file===
<?php
class Svc {
    public function helper(): void {}
    public function run(): void {
        $this->helper();
    }
}
===expect===
