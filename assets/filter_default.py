# The default filter hides subtasks, unless viewed from their parent task

ptags = task.get_tags_with_name("subtask_of")

if len(ptags) == 0:
	# Subtask of nothing => show
	return True
else:
	# Subtask of something => check if we are under its parent(s)
	for ptag in ptags:
		if ptag.value == parent:
			# We are under its parent => show
			return True
	
	# We are not under its parent(s) => hide
	return False
