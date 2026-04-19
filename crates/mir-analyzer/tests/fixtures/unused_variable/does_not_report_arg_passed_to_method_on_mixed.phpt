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
UnusedParam: Parameter $a is never used
