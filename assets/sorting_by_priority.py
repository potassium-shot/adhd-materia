# Allows sorting by priority
# Searches for a tag "priority"
# The tag may have values like "SSS", "S", "A", "B", "C", "D", "E", "F"
# As well as an optional modifier like +, ++, +++, -, --, ---
#
# Example: "C", "A+", "B--", "SS+++"

prios = {"SSS": 0, "SS": 10, "S": 20, "A": 30, "B": 40, "C": 50, "D": 60, "E": 70, "F": 80}
modif = {"": 0, "+": -1, "++": -2, "+++": -3, "-": 1, "--": 2, "---": 3}

prio_tag = task.get_tag_with_name("priority")

if prio_tag != None:
	prio_str = ""
	modif_str = ""
	for c in prio_tag.value:
		if ord(c) >= ord('A') and ord(c) <= ord('Z'):
			prio_str += c
		else:
			modif_str += c

	return prios[prio_str] + modif[modif_str]
else:
	return 99999