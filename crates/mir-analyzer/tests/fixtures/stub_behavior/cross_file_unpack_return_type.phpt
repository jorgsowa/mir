===file:Protocol.php===
<?php
/**
 * @return array<int, mixed>|false
 */
function parsePacket(string $data): array|false {
    return unpack('N*', $data);
}
===file:Main.php===
<?php
$result = parsePacket("\x00\x00\x00\x01");
if ($result !== false) {
    $first = $result[1];
}
===expect===
