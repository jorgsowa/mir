===description===
Tainted llm prompt through function
===config===
suppress=MixedArrayAccess,UnusedParam
===file===
<?php
class LlmAgent {
    /** @taint-sink llm_prompt $prompt */
    public function prompt(string $prompt): string {
        return "";
    }
}

function buildPrompt(string $userInput): string {
    return "Tell me about " . $userInput;
}

$agent = new LlmAgent();
$agent->prompt(buildPrompt((string) $_GET["topic"]));
===expect===
