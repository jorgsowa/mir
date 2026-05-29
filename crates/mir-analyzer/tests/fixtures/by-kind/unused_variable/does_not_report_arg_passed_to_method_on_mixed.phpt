===description===
does not report arg passed to method on mixed
===file===
<?php
class Foo {
    public function handle(): void {
        $ctx = ['key' => 'value'];
        $this->doSomething($ctx);
    }

    private function doSomething(array $a): void {}
}
===expect===
UnusedParam@8:34-8:42: Parameter $a is never used
