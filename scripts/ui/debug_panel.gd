class_name DebugPanel
extends Control
## Pannello di debug: slider per tutti i parametri di CarTuning, modificabili mentre si guida.
## Sta sul lato destro così il pollice sinistro può continuare a sterzare.

var tuning: CarTuning
var defaults: CarTuning  ## copia dei default da ripristinare

var _toggle: Button
var _panel: PanelContainer
var _sliders: Dictionary = {}  ## proprietà -> HSlider
var _labels: Dictionary = {}  ## proprietà -> Label
var _export_text: TextEdit
var _status: Label


## Costruisce l'interfaccia. Chiamato da main.gd dopo aver assegnato i parametri.
func setup(t: CarTuning, d: CarTuning) -> void:
	tuning = t
	defaults = d
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_IGNORE

	_toggle = Button.new()
	_toggle.text = "⚙"
	_toggle.add_theme_font_size_override("font_size", 34)
	_toggle.custom_minimum_size = Vector2(72, 72)
	_toggle.set_anchors_preset(Control.PRESET_TOP_RIGHT)
	_toggle.offset_left = -84
	_toggle.offset_right = -12
	_toggle.offset_top = 12
	_toggle.offset_bottom = 84
	_toggle.pressed.connect(func() -> void: _panel.visible = not _panel.visible)
	add_child(_toggle)

	_panel = PanelContainer.new()
	_panel.visible = false
	_panel.anchor_left = 0.55
	_panel.anchor_right = 1.0
	_panel.anchor_top = 0.0
	_panel.anchor_bottom = 1.0
	_panel.offset_top = 96
	_panel.offset_bottom = -8
	_panel.offset_right = -8
	_panel.self_modulate = Color(1, 1, 1, 0.92)
	add_child(_panel)

	var root := VBoxContainer.new()
	_panel.add_child(root)

	var buttons := HBoxContainer.new()
	root.add_child(buttons)
	buttons.add_child(_make_button("Ripristina", _on_reset))
	buttons.add_child(_make_button("Copia valori", _on_export))
	_status = Label.new()
	root.add_child(_status)

	var scroll := ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	root.add_child(scroll)
	var list := VBoxContainer.new()
	list.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(list)

	var group := ""
	for p in CarTuning.PARAMS:
		if p[0] != group:
			group = p[0]
			var header := Label.new()
			header.text = "— %s —" % group
			header.add_theme_font_size_override("font_size", 22)
			list.add_child(header)
		var label := Label.new()
		list.add_child(label)
		var slider := HSlider.new()
		slider.min_value = p[3]
		slider.max_value = p[4]
		slider.step = p[5]
		slider.value = tuning.get(p[1])
		slider.custom_minimum_size = Vector2(0, 44)
		slider.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		slider.value_changed.connect(_on_slider.bind(p[1]))
		list.add_child(slider)
		_sliders[p[1]] = slider
		_labels[p[1]] = label
		_update_label(p[1])

	_export_text = TextEdit.new()
	_export_text.editable = false
	_export_text.custom_minimum_size = Vector2(0, 360)
	list.add_child(_export_text)


## true se il punto (coordinate schermo) cade sul pulsante o sul pannello aperto.
func blocks(p: Vector2) -> bool:
	if _toggle.get_global_rect().has_point(p):
		return true
	return _panel.visible and _panel.get_global_rect().has_point(p)


func _make_button(text: String, callback: Callable) -> Button:
	var b := Button.new()
	b.text = text
	b.custom_minimum_size = Vector2(0, 56)
	b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	b.pressed.connect(callback)
	return b


func _on_slider(value: float, prop: String) -> void:
	tuning.set(prop, value)
	_update_label(prop)


func _update_label(prop: String) -> void:
	for p in CarTuning.PARAMS:
		if p[1] == prop:
			_labels[prop].text = "%s: %s" % [p[2], str(snappedf(tuning.get(prop), 0.001))]
			return


func _on_reset() -> void:
	tuning.copy_from(defaults)
	for p in CarTuning.PARAMS:
		_sliders[p[1]].set_value_no_signal(tuning.get(p[1]))
		_update_label(p[1])
	_status.text = "Valori ripristinati"


func _on_export() -> void:
	var text := tuning.to_text()
	_export_text.text = text
	print("--- TUNING ---\n", text)
	DisplayServer.clipboard_set(text)
	_status.text = "Copiati negli appunti"
