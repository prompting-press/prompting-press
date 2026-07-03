"""Use Prompting Press prompts with CrewAI (Python).

CrewAI is field-based: an ``Agent`` takes ``role`` / ``goal`` / ``backstory``
strings and a ``Task`` takes ``description`` / ``expected_output`` strings.
There is no message array. So the bridge is direct assignment: render each
prompt with Prompting Press and pass the RENDERED string (the ``render(...).text``
field — NOT ``Prompt.body``, which is the raw un-rendered template) into the
matching field.

Note: do NOT ALSO pass the same variables to ``crew.kickoff(inputs=...)``.
CrewAI runs its own ``{placeholder}`` interpolation over these strings; if
Prompting Press already substituted them, the ``{placeholder}`` text no longer
exists and re-interpolation is at best redundant and at worst confusing. Render
once, with Prompting Press, and hand CrewAI final strings. Standalone.
"""

from crewai import Agent, Task
from prompting_press import Prompt
from pydantic import BaseModel


class RoleVars(BaseModel):
    domain: str


class GoalVars(BaseModel):
    topic: str


# Prompt definitions, each rendered to a final string with typed vars.
role_prompt = Prompt(
    {
        "name": "agent-role",
        "role": "system",
        "body": "Senior {{ domain }} researcher",
        "variables": {"domain": {"type": "string", "trusted": True}},
    }
)
goal_prompt = Prompt(
    {
        "name": "task-goal",
        "role": "user",
        "body": "Write a concise brief on {{ topic }}.",
        "variables": {"topic": {"type": "string", "trusted": False}},
    }
)

# Render -> use the .text field (the rendered output string).
rendered_role = role_prompt.render(RoleVars, data={"domain": "robotics"}).text
rendered_goal = goal_prompt.render(GoalVars, data={"topic": "actuator safety"}).text

# Assign rendered strings directly to CrewAI's fields. `llm=` is set so the
# Agent constructs offline without needing a live provider probe; we never call
# crew.kickoff(), so no network / no API key is used.
agent = Agent(
    role=rendered_role,
    goal=rendered_goal,
    backstory="You have published widely on industrial robotics.",
    llm="gpt-4o",
)
task = Task(
    description=rendered_goal,
    expected_output="A one-paragraph brief.",
    agent=agent,
)

print(agent.role)  # "Senior robotics researcher"
print(task.description)  # "Write a concise brief on actuator safety."

# --- assertions (this file is executed by CI) ---

# The rendered .text strings are assigned verbatim to CrewAI's fields.
assert agent.role == "Senior robotics researcher"
assert agent.goal == "Write a concise brief on actuator safety."
assert task.description == "Write a concise brief on actuator safety."
assert task.expected_output == "A one-paragraph brief."

# Guard against the .body-vs-.text trap: .body is the RAW template, .text is
# rendered. A recipe that used .body would ship un-substituted {{ }} text.
assert role_prompt.body == "Senior {{ domain }} researcher"
assert rendered_role == "Senior robotics researcher"
assert "{{" not in agent.role
