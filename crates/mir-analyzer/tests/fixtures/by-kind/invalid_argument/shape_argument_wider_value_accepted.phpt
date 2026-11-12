===description===
A shape value with more keys than a sealed shape parameter declares is accepted when it is not an array literal, such as a built-up array or a nested shape read from another array.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

final class Cache
{
    /** @param array{initial_age: int, stored_at: int} $data */
    private function age(array $data): int
    {
        return $data['initial_age'] + $data['stored_at'];
    }

    public function build(string $content, int $status): int
    {
        $data = [
            'content' => $content,
            'status_code' => $status,
            'initial_age' => 0,
        ];
        $data['stored_at'] = time();

        return $this->age($data);
    }

    /** @param array{entry: array{initial_age: int, stored_at: int, headers: list<string>}} $wrapped */
    public function nested(array $wrapped): int
    {
        return $this->age($wrapped['entry']);
    }

    /** @param array{initial_age: string, stored_at: int, headers: list<string>} $bad */
    public function wrongValue(array $bad): int
    {
        return $this->age($bad);
        //                ^^^^ InvalidArgument: Argument $data of age() expects 'array{'initial_age': int, 'stored_at': int}', got 'array{'initial_age': string, 'stored_at': int, 'headers': list<string>}'
    }
}
