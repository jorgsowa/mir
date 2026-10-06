===description===
dataProvider credit does not leak to an unrelated class or a genuinely unused method
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
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
//  ^ +2:5 UnusedMethod: Private method FooTest::unrelatedhelper() is never called
        return [];
    }
}

class OtherTest {
    private static function providecases(): array {
//  ^ +2:5 UnusedMethod: Private method OtherTest::providecases() is never called
        return [];
    }
}
