===source===
<?php
class Foo {
    public function handle(): void {
        $ctx = ['key' => 'value'];
        $this->doSomething($ctx);
    }

    private function doSomething(array $a): void {}
}
===expect===
MixedMethodCall: $this->doSomething($ctx)
UnusedParam: $a
