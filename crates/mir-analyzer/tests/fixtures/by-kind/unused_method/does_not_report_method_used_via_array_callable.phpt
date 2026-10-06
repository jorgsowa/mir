===description===
A private method used only through the `[$this, 'method']` array-callable
literal (passed to call_user_func, here invoked directly) must not be
reported unused.
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
class Foo {
    private function helper(): void {}

    public function run(): void {
        call_user_func([$this, 'helper']);
    }
}
