===description===
nested class method parameter not undefined no error
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
===ignore===
TODO
