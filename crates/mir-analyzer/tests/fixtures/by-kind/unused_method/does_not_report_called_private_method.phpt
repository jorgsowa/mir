===description===
does not report called private method
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
class Foo {
    public function run(): void {
        $this->helper();
    }

    private function helper(): void {}
}
