===file===
<?php
/**
 * @param string|null $value
 * @return ($value is null ? array<string> : non-empty-string)
 */
function wrap(?string $value): array|string
{
    if ($value === null) {
        return [];
    }
    return $value . '!';
}
===expect===
