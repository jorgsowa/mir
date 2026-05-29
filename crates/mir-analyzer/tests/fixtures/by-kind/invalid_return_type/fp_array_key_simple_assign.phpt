===description===
Array key type preserved when assigning directly ($arr[$key] = $value)
===file===
<?php

class DtoRule {}

/**
 * @param array<string, DtoRule> $dtoRules
 * @return array<string, DtoRule>
 */
function buildMap(array $dtoRules): array
{
    $result = [];
    foreach ($dtoRules as $fieldName => $rule) {
        $result[$fieldName] = $rule;
    }
    return $result;
}
===expect===
