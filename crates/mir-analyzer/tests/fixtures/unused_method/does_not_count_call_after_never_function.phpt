===config===
find_dead_code=true
===file===
<?php
function stop(): never {
    throw new RuntimeException('stop');
}

class Foo {
    public function run(): void {
        stop();
        $this->helper();
    }

    private function helper(): void {}
}
===expect===
UnreachableCode: Unreachable code detected
UnusedMethod: Private method Foo::helper() is never called
