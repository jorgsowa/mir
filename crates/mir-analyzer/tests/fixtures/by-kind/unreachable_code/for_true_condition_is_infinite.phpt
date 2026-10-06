===description===
A for loop with a literal true condition never falls through.
===file===
<?php
class Runner {
    public function run(int $limit): int {
        for ($i = 1; true; ++$i) {
            if ($i > $limit) {
                return 0;
            }
        }
    }
}
