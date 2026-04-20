===source===
<?php
function outer(): void {
    class Inner {
        public function process(string $data): string {
            return $data;
        }
    }
}
===expect===
