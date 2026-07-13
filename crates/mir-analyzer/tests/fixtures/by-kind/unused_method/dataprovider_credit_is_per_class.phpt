===description===
dataProvider credit does not leak to an unrelated class or a genuinely unused method
===config===
suppress=
===file===
<?php
class FooTest {
    /**
     * @dataProvider provideCases
     */
    public function testAdd(int $a, int $b): void {
        echo $a + $b;
    }

    private static function providecases(): array {
        return [[1, 2]];
    }

    private static function unrelatedHelper(): array {
        return [];
    }
}

class OtherTest {
    private static function providecases(): array {
        return [];
    }
}
===expect===
UnusedMethod@14:4-16:5: Private method FooTest::unrelatedhelper() is never called
UnusedMethod@20:4-22:5: Private method OtherTest::providecases() is never called
