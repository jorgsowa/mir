===description===
template narrowing combined with null check
===file===
<?php
class DataSet {
    public string $name;
}

/**
 * @template TData as DataSet|null
 * @param TData $data
 */
function processData(DataSet|null $data): void {
    if ($data === null) {
        return;
    }
    echo $data->name;
}

/**
 * Combining instanceof with null check
 * @template TValue as string|int|null
 * @param TValue $value
 */
function handleValue(string|int|null $value): void {
    if ($value === null) {
        return;
    }

    if (is_string($value)) {
        echo strlen($value);
    } else {
        echo $value * 2;
    }
}
===expect===
MissingConstructor@2:0-2:15: Class DataSet has uninitialized properties but no constructor
