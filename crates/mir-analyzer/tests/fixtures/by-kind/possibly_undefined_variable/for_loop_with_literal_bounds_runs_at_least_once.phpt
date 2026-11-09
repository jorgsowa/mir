===description===
A for loop whose init and condition are int literals that hold on entry runs its body at least once
===file===
<?php
function firstName(array $used): string {
    for ($attempt = 0; $attempt < 50; $attempt++) {
        $name = 'n' . $attempt;
        if (!isset($used[$name])) {
            break;
        }
    }
    return $name;
}
function countdown(): int {
    for ($i = 10; $i >= 1; $i--) {
        $last = $i;
    }
    return $last;
}
