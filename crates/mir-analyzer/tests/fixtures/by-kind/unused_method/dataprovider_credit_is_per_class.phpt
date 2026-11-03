===description===
dataProvider credit does not leak to an unrelated class or a genuinely unused method
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
class FooTest {
//<^ +15:1 UnusedClass: Class FooTest is never referenced
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
//  ^ +2:5 UnusedMethod: Private method FooTest::unrelatedHelper() is never called
        return [];
    }
}

class OtherTest {
//<^ +4:1 UnusedClass: Class OtherTest is never referenced
    private static function providecases(): array {
//  ^ +2:5 UnusedMethod: Private method OtherTest::providecases() is never called
        return [];
    }
}
