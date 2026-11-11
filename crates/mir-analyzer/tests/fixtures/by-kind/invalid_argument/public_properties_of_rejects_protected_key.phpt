===description===
`public-properties-of<User>` is sealed to the public properties: a key naming a protected or private property is `InvalidArgument`.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

final class User {
    public string $name = '';
    protected int $age = 0;
    private string $secret = '';

    public function describe(): string {
        return $this->name . $this->age . $this->secret;
    }
}

/** @param public-properties-of<User> $data */
function acceptsPublic(array $data): void {}

acceptsPublic(['name' => 'Ada']);
acceptsPublic(['name' => 'Ada', 'age' => 42]);
//            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $data of acceptsPublic() expects 'array{'name': string}', got 'array{'name': "Ada", 'age': 42}'
acceptsPublic(['name' => 'Ada', 'secret' => 'x']);
//            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $data of acceptsPublic() expects 'array{'name': string}', got 'array{'name': "Ada", 'secret': "x"}'
