===description===
A closed shape past the key cap still generalizes to its written value types
===file===
<?php
function take(int $id): void { echo $id; }

function closed_shape(): void {
    $row = ['id' => 1];
    $row['k1'] = 's';
    $row['k2'] = 's';
    $row['k3'] = 's';
    $row['k4'] = 's';
    $row['k5'] = 's';
    $row['k6'] = 's';
    $row['k7'] = 's';
    $row['k8'] = 's';
    $row['k9'] = 's';
    take($row['k9']);
//       ^^^^^^^^^^ PossiblyInvalidArgument: Argument $id of take() expects 'int', possibly different type '"s"|1' provided
}
