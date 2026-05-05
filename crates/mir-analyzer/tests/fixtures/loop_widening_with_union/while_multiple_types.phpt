===description===
whileMultipleTypes
===file===
<?php
$result = null;
$i = 0;
while ($i < 3) {
    if ($i === 0) {
        $result = "string";
    } else {
        $result = 42;
    }
    $i++;
}
// After loop, $result should be string|int|null, not mixed
echo $result;
===expect===
