define e = Character("Eileen", color="#c8ffc8")
define s = Character("Student", color="#c8c8ff")

label start:
    scene bg classroom
    show eileen happy at left
    show student neutral at right

    e "Welcome! Today we are going to learn something new."
    s "I'm ready! What's the topic?"

    # LLM insert generated content here

    return