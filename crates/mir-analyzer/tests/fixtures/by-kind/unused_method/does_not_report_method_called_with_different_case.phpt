===description===
Method names are case-insensitive
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
final class A {
    private function helperMethod(): void {}
    private static function staticHelper(): void {}

    public function run(): void {
        $this->HELPERMETHOD();
//             ^^^^^^^^^^^^ WrongCaseMethod: Method name 'A::HELPERMETHOD' has incorrect casing; use 'helperMethod'
        self::STATICHELPER();
//            ^^^^^^^^^^^^ WrongCaseMethod: Method name 'A::STATICHELPER' has incorrect casing; use 'staticHelper'
    }
}

(new A())->run();
