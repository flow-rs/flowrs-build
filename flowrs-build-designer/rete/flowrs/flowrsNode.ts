import {ClassicPreset, ClassicPreset as Classic, NodeEditor} from 'rete';
import {
    type ConstructorDefinition,
    type ConstructorDescription,
    type Type,
    type TypeDescription
} from "~/repository/modules/packages";
import {DropdownControl, type Schemes} from "~/rete/flowrs/editor";

const socket = new Classic.Socket('socket');

export class FlowrsNode extends Classic.Node<
    Record<string, ClassicPreset.Socket>,
    Record<string, ClassicPreset.Socket>,
    Record<
        string,
        | DropdownControl
        | ClassicPreset.InputControl<"number">
        | ClassicPreset.InputControl<"text">>> {
    width = 470;
    height = 60;
    public node_data: string | undefined;
    public typeParameters: Map<string, string> = new Map();
    private inAndOutputToTypeParameterMap: Map<string, string> = new Map();
    public constructor_type: string = "New";
    public fullTypeName: string;

    private editor: NodeEditor<Schemes>;

    constructor(name: string,
                fullTypeName: string,
                data: { [key: string]: any } | null,
                constructor_type: string,
                typeParameters: { [key: string]: string } | null,
                allPossibleTypes: Map<string, Type>,
                editor: NodeEditor<Schemes>
    ) {
        super(name);

        this.editor = editor;
        this.fullTypeName = fullTypeName;
        let typeDefinition: Type = allPossibleTypes.get(this.fullTypeName)!
        this.constructor_type = constructor_type;
        this.setNodeData(data);
        this.setTypeParameters(typeParameters);
        let constructorDescription = this.getConstructorDescription(typeDefinition.constructors);
        this.addControlInputs(constructorDescription);
        this.addDropdownControlsForGenericTypeParameters(typeDefinition.type_parameters, constructorDescription, allPossibleTypes);
        this.addInputs(typeDefinition);
        this.addOutputs(typeDefinition);
    }

    private setNodeData(data: { [key: string]: any } | null) {
        if (!data) {
            return
        }

        this.node_data = JSON.stringify(data, null, 2);
    }

    private setTypeParameters(typeParameters: { [p: string]: string } | null) {
        if (!typeParameters) {
            return;
        }

        for (const typeParametersKey in typeParameters) {
            this.typeParameters.set(typeParametersKey, typeParameters[typeParametersKey]);
        }

        console.log("Type parameters set:", this.typeParameters);
    }

    private getConstructorDescription(constructorDefinition: ConstructorDefinition) {
        if (!constructorDefinition) {
            return;
        }
        console.log(constructorDefinition, this.constructor_type)
        let currentConstructor = constructorDefinition[this.constructor_type];
        if (!currentConstructor) {
            console.error("Current constructor doesnt exist", this.constructor_type, constructorDefinition)
            return;
        }
        if (typeof currentConstructor == "string") {
            console.error("Current constructor is a string constructor", this.constructor_type, constructorDefinition)
            return;
        }
        let recordKeys = Object.keys(currentConstructor);
        if (recordKeys.length > 1) {
            console.error("Package endpoint malformed output?", currentConstructor);
        }
        return currentConstructor[recordKeys[0]];
    }

    private addControlInputs(constructor: ConstructorDescription | undefined) {
        if (!constructor?.arguments) {
            return
        }
        for (const argument of constructor.arguments) {
            // TODO streng genommen müsste bei Json constructorDescription ein TextInputField hinzu gefügt werden genau dann wenn Key == Json und gewählter generic typ den constructor value FromJson hat --> "Json": "FromJson"
            // ist aber ein riesen aufwand mit reaktivität vom dropdown auf mögliche x inputfelder --> darum nur auf json fürs erste prüfen
            if (argument.construction.Constructor == "Json") {
                this.addControl(
                    'data',
                    new Classic.InputControl('text', {
                        initial: this.node_data, readonly: false, change: value => {
                            this.node_data = value;
                        }
                    })
                );
                this.height += 38;
            }
        }
    }

    private addDropdownControlsForGenericTypeParameters(type_parameters: string[] | undefined, constructorDescription: undefined | ConstructorDescription, allPossibleTypes: Map<string, Type>) {
        if (!type_parameters) {
            return;
        }

        for (const typeParameter of type_parameters) {
            let constructorNameToFilterFor: string | null = this.getConstructorNameToFilterFor(constructorDescription, typeParameter);
            let possibleTypes: [string, Type][] = this.chooseMethodAndGetPossibleTypes(constructorNameToFilterFor, allPossibleTypes);
            let possibleTypeNames: string[] = possibleTypes.map(([typeName, typeDefinition]) => typeName);
            this.addControl(
                typeParameter,
                new DropdownControl(typeParameter, possibleTypeNames, this.typeParameters.get(typeParameter), (selectedTypeName: string) => {
                    this.typeParameters.set(typeParameter, selectedTypeName);

                    for (let connection of this.editor.getConnections()) {
                        if (this.id == connection.target || this.id == connection.source) {
                            this.editor.removeConnection(connection.id);
                        }
                    }
                })
            );
            this.height += 90;
        }
    }

    private getConstructorNameToFilterFor(constructorDescription: ConstructorDescription | undefined, typeParameter: string): string | null {
        if (!constructorDescription?.arguments) {
            return null;
        }
        let restrictingConstructorType: string | null = null;
        for (const argument of constructorDescription.arguments) {
            console.log("Checking for restrictingConstructorType for", typeParameter, "\n", argument)
            if (argument.type.Generic && argument.type.Generic.name == typeParameter && argument.construction.Constructor) {
                restrictingConstructorType = argument.construction.Constructor;
                break;
            }
        }
        return restrictingConstructorType;
    }

    private chooseMethodAndGetPossibleTypes(constructorNameToFilterFor: null | string, allPossibleTypes: Map<string, Type>): [string, Type][] {
        if (constructorNameToFilterFor) {
            let possibleTypes = this.getFilteredTypesList(constructorNameToFilterFor, allPossibleTypes);
            console.log("RestrictingConstructorType resulted into filtered list", possibleTypes);
            return possibleTypes;
        } else {
            let possibleTypes = Array.from(allPossibleTypes.entries());
            console.log("Full list available", possibleTypes);
            return possibleTypes;
        }
    }

    // Man müsste nach Traits filtern --> Information noch nicht im code enthalten
    private getFilteredTypesList(constructorNameToFilterFor: string, allPossibleTypes: Map<string, Type>): [string, Type][] {
        let filteredTypes: [string, Type][] = [];
        for (const possibleType of allPossibleTypes) {
            let typeDefinition = possibleType[1];
            let constructorDefinition = typeDefinition.constructors[constructorNameToFilterFor];
            if (constructorDefinition) {
                filteredTypes.push(possibleType);
                continue;
            }
        }
        return filteredTypes;
    }

    private addOutputs(types: Type) {
        for (let outputName in types.outputs) {
            let typeDescription = types.outputs[outputName].type;
            let typeName = this.getTypeName(typeDescription);
            this.addOutput(outputName, new Classic.Output(socket, outputName + ':' + typeName, false));
            this.inAndOutputToTypeParameterMap.set(outputName, typeName);
            this.height += 36;
        }
    }

    private addInputs(types: Type) {
        for (let inputName in types.inputs) {
            let typeDescription = types.inputs[inputName].type;
            let typeName = this.getTypeName(typeDescription);
            this.addInput(inputName, new Classic.Input(socket, inputName + ':' + typeName, false));
            this.inAndOutputToTypeParameterMap.set(inputName, typeName);
            this.height += 36;
        }
    }

    private getTypeName(typeDescription: TypeDescription) {
        if (typeDescription.Generic) {
            return typeDescription.Generic.name;
        } else if (typeDescription.Type) {
            return typeDescription.Type.name;
        } else {
            console.error("Type not supported", typeDescription)
        }
        return "unknown";
    }

    public getTypeForKey(key: string): string | undefined {
        let typeName = this.inAndOutputToTypeParameterMap.get(key);
        if (!typeName) {
            return;
        }
        let genericResolutionType = this.typeParameters.get(typeName);
        if (genericResolutionType) {
            return genericResolutionType
        }
        return typeName;
    }

    data() {
        const value = this.node_data;
        return {value,};
    }
}