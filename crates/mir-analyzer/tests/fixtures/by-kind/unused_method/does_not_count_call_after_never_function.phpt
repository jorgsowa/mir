===description===
does not count call after never function
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
UnreachableCode@9:9-9:25: Unreachable code detected
UnusedMethod@12:4-12:38: Private method Foo::helper() is never called
