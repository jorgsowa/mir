===description===
nested class method parameter not undefined no error
===config===
suppress=MixedReturnStatement
===file===
<?php
function outer(): void {
    class Inner {
        public function process(string $data): string {
            return $data;
        }
    }
}
===expect===
