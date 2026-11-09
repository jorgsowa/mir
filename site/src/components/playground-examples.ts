export type PlaygroundExample = { label: string; code: string };

export const EXAMPLES: Record<string, PlaygroundExample> = {
  'type-mismatch': {
    label: 'Type mismatch',
    code: `<?php

function add(int $a, int $b): int {
    return $a + $b;
}

$result = add("not a number", 42);
`,
  },
  'undefined-method': {
    label: 'Undefined method',
    code: `<?php

class User {
    public function getEmail(): string { return ''; }
}

$user = new User();
echo $user->getName();
`,
  },
  'nullability': {
    label: 'Null method call',
    code: `<?php

class User {
    public function name(): string { return ''; }
}

$user = null;
echo $user->name();
`,
  },
  'array-access': {
    label: 'Array access',
    code: `<?php

$counts = ['a' => 1, 'b' => 2];
echo $counts['missing'] + 1;
`,
  },
  'dead-code': {
    label: 'Dead code',
    code: `<?php

function greet(string $name): string {
    return "Hello, $name";
    echo 'never runs';
}

function helper(): void {
    $unused = 42;
}
`,
  },
  'taint': {
    label: 'Taint (SQL)',
    code: `<?php

function run_query(\\mysqli $db): void {
    mysqli_query($db, $_GET['sql']);
}
`,
  },
  'generics': {
    label: 'Generics',
    code: `<?php

/**
 * @template T of int
 * @param T $value
 * @return T
 */
function identity($value) {
    return $value;
}

identity("not an int");
`,
  },
  'controller': {
    label: 'Controller',
    code: `<?php

final class User { public function __construct(public string $email) {} }

interface Users { public function find(int $id): ?User; }

final class UserController
{
    public function __construct(private Users $users, private PDO $db) {}

    public function show(): string
    {
        $user = $this->users->find('42');
        $this->db->query("SELECT * FROM t WHERE q = '{$_GET['q']}'");

        return $user->getEmail();
    }
}
`,
  },
};
