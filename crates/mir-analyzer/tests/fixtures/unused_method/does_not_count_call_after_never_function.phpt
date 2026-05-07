===description===
does not count call after never function
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
MissingThrowsDocblock@3:4: Exception RuntimeException is thrown but not declared in @throws
UnreachableCode@9:8: Unreachable code detected
UnusedMethod@1:0: Private method Foo::helper() is never called
