import {ClassicPreset, ClassicPreset as Classic} from 'rete';
import {
    type ConstructorDefinition,
    type ConstructorDescription,
    type TypeDefinition,
    type TypeDescription
} from "~/repository/modules/packages";
import {DropdownControl} from "~/rete/flowrs/editor";

const socket = new Classic.Socket('socket');

export class FlowrsNode extends Classic.Node<
    Record<string, ClassicPreset.Socket>,
    Record<string, ClassicPreset.Socket>,
    Record<
        string,
        | DropdownControl
        | ClassicPreset.InputControl<"number">
        | ClassicPreset.InputControl<"text">>> {
    width = 500;
    height = 140;
    public node_data: string | undefined;
    public genericsResolutionMap: Map<string, string> = new Map();
    private keyToTypeParameterMap: Map<string, string> = new Map();
    public constructor_type: string = "New";

    constructor(name: string,
                typeDefinition: TypeDefinition,
                data: { [key: string]: any } | null,
                constructor_type: string,
                typeParameters: { [key: string]: string } | null,
                allPossibleTypes: Map<string, TypeDefinition>
    ) {
        // TODO eindeutige namen
        super(name);

        this.constructor_type = constructor_type;
        this.setNodeData(data);
        this.setTypeParameters(typeParameters);
        let constructorDescription = this.getConstructorDescription(typeDefinition.constructors);
        // TODO bei Json constructorDescription füg ich ein TextInputField hinzu --> richtig ? Prüfen abhängig vom gewählten typen Constructor --> Key == Json und Value == FromJson
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
            this.genericsResolutionMap.set(typeParametersKey, typeParameters[typeParametersKey]);
        }

        console.log("Type parameters set:", this.genericsResolutionMap);
    }

    private getConstructorDescription(constructorDefinition: ConstructorDefinition) {
        let currentConstructor: Record<string, ConstructorDescription> | undefined;
        if (this.constructor_type == "New") {
            currentConstructor = constructorDefinition.New;
        } else if (this.constructor_type == "NewWithToken") {
            currentConstructor = constructorDefinition.NewWithToken;
        }
        if (!currentConstructor) {
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
            if (argument.construction.Constructor == "Json") {
                this.addControl(
                    'data',
                    new Classic.InputControl('text', {
                        initial: this.node_data, readonly: false, change: value => {
                            this.node_data = value;
                        }
                    })
                );
            }
        }
    }

    private addDropdownControlsForGenericTypeParameters(type_parameters: string[] | undefined, constructorDescription: undefined | ConstructorDescription, allPossibleTypes: Map<string, TypeDefinition>) {
        if (!type_parameters) {
            return;
        }

        for (const typeParameter of type_parameters) {
            let constructorNameToFilterFor: string | null = this.getConstructorNameToFilterFor(constructorDescription, typeParameter);
            let possibleTypes: [string, TypeDefinition][] = this.chooseMethodAndGetPossibleTypes(constructorNameToFilterFor, allPossibleTypes);
            this.addControl(
                typeParameter,
                new DropdownControl(typeParameter, possibleTypes, (selectedTypeName: string) => {
                    this.genericsResolutionMap.set(typeParameter, selectedTypeName);
                    // TODO remove not valid inputs OR start somehow middleware logic over everything ?
                })
            );
            this.height += 75;
        }
    }

    private getConstructorNameToFilterFor(constructorDescription: ConstructorDescription | undefined, typeParameter: string): string | null {
        if (!constructorDescription?.arguments) {
            return null;
        }
        // TODO @mafried: gibts immer nur eine möglichkeit ?
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

    private chooseMethodAndGetPossibleTypes(constructorNameToFilterFor: null | string, allPossibleTypes: Map<string, TypeDefinition>): [string, TypeDefinition][] {
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
    private getFilteredTypesList(constructorNameToFilterFor: string, allPossibleTypes: Map<string, TypeDefinition>): [string, TypeDefinition][] {
        let filteredTypes: [string, TypeDefinition][] = [];
        for (const possibleType of allPossibleTypes) {
            let typeDefinition = possibleType[1];
            let constructorDefinition = typeDefinition.constructors;

            switch (constructorNameToFilterFor) {
                case "Json":
                    if (constructorDefinition.Json) {
                        filteredTypes.push(possibleType);
                        continue;
                    }
                    break;
                case "Default":
                    if (constructorDefinition.Default) {
                        filteredTypes.push(possibleType);
                        continue;
                    }
                    break;
                case "New":
                    if (constructorDefinition.New) {
                        filteredTypes.push(possibleType);
                        continue;
                    }
                    break;
                case "NewWithToken":
                    if (constructorDefinition.NewWithToken) {
                        filteredTypes.push(possibleType);
                        continue;
                    }
                    break;
                default:
                    console.error("Error in the filtering for types based on restriction of constructor argument constructor. ConstructorDefinitions don't know about:", constructorNameToFilterFor, "This is probably a business logic error and");
            }
        }
        return filteredTypes;
    }

    private addOutputs(types: TypeDefinition) {
        for (let outputName in types.outputs) {
            let typeDescription = types.outputs[outputName].type;
            let typeName = this.getTypeName(typeDescription);
            this.addOutput(outputName, new Classic.Output(socket, outputName + ':' + typeName, false));
            this.keyToTypeParameterMap.set(outputName, typeName);
        }
    }

    private addInputs(types: TypeDefinition) {
        for (let inputName in types.inputs) {
            let typeDescription = types.inputs[inputName].type;
            let typeName = this.getTypeName(typeDescription);
            this.addInput(inputName, new Classic.Input(socket, inputName + ':' + typeName, false));
            this.keyToTypeParameterMap.set(inputName, typeName);
            this.height += 20;
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
        let typeName = this.keyToTypeParameterMap.get(key);
        if (!typeName) {
            return;
        }
        let genericResolutionType = this.genericsResolutionMap.get(typeName);
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